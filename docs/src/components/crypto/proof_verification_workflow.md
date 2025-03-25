<!-- Workflow for proof verification -->
<!-- Original file: docs/src/proof_verification_workflow.md -->

# Proof Verification Workflow

This document describes the proof verification workflow for facts in the Causality system, particularly focusing on how the new `FactType` system integrates with zero-knowledge proofs.

## Overview

Proof verification is a crucial component of the Causality system that ensures facts can be cryptographically verified. The system supports various proof mechanisms, including:

1. Merkle proofs for block inclusion
2. Signatures for authenticated facts
3. Consensus proofs for cross-domain validation

## Proof Components

Each verifiable fact in the system consists of three core components:

1. **The Fact Data**: The actual information being claimed (e.g., account balance, transaction details)
2. **The Proof Data**: Cryptographic evidence supporting the fact's validity
3. **The Verification Context**: Additional information needed to verify the proof

## Fact-Proof Relationship

In the new `FactType` system, proofs are represented using the following structures:

```rust
// Representation of a proof within the fact system
pub struct FactProof {
    pub proof_type: ProofType,
    pub proof_data: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

// Types of proofs supported by the system
pub enum ProofType {
    MerkleProof,
    Signature,
    ZkProof,
    Consensus,
    Custom(String),
}
```

Each `FactType` variant can include a `proof` field containing a `FactProof` instance. For example, a `RegisterFact` might include a ZK proof demonstrating valid state transitions.

## Verification Process

The verification workflow consists of the following steps:

1. **Fact Observation**: A fact is observed from a domain adapter
2. **Proof Extraction**: The proof is extracted from the fact
3. **Verifier Selection**: The appropriate verifier is selected based on the proof type
4. **Verification**: The proof is verified against the fact data
5. **Result Handling**: The system handles the verification result

### Detailed Workflow

#### 1. Fact Observation

Facts are observed through domain adapters which implement the `DomainAdapter` trait:

```rust
let fact_query = FactQuery {
    domain_id: domain_id.clone(),
    fact_type: "register_create".to_string(),
    parameters: params,
    block_height: None,
    block_hash: None,
    timestamp: None,
};

let fact = domain_adapter.observe_fact(fact_query).await?;
```

#### 2. Proof Extraction

The proof is extracted from the fact based on its type:

```rust
match fact {
    FactType::RegisterFact(RegisterFact::RegisterCreation { proof, .. }) => {
        // Extract the proof for verification
        if let Some(proof) = proof {
            // Process the proof
        }
    },
    // Other fact types...
    _ => { /* Handle other fact types */ }
}
```

#### 3. Verifier Selection

The system selects an appropriate verifier based on the proof type:

```rust
let verifier = match proof.proof_type {
    ProofType::ZkProof => verifier_registry.get_verifier("zk"),
    ProofType::MerkleProof => verifier_registry.get_verifier("merkle"),
    ProofType::Signature => verifier_registry.get_verifier("signature"),
    ProofType::Consensus => verifier_registry.get_verifier("consensus"),
    ProofType::Custom(ref name) => verifier_registry.get_verifier(name),
};
```

#### 4. Verification

The verifier checks the proof against the fact data:

```rust
let verification_result = verifier.verify(&fact, &proof).await?;

match verification_result {
    VerificationResult::Valid => {
        // Fact is verified
    },
    VerificationResult::Invalid(reason) => {
        // Verification failed
    },
    VerificationResult::Indeterminate => {
        // Verification is inconclusive
    }
}
```

#### 5. Result Handling

Based on the verification result, the system can:

- Accept the fact as verified
- Reject the fact
- Request additional proof
- Log the verification attempt

## Register Fact Verification

Register facts (creation, update, transfer) have a specialized verification workflow:

### Register Creation Verification

When a register is created, the proof must demonstrate:

1. The register ID is unique
2. The creator has authority to create the register
3. The initial state is valid

```rust
// Example code for verifying a register creation proof
match fact {
    FactType::RegisterFact(RegisterFact::RegisterCreation { 
        register_id, 
        owner, 
        register_type, 
        initial_value, 
        proof, 
        .. 
    }) => {
        if let Some(proof) = proof {
            let verifier = register_verifier_registry.get_verifier("register_creation");
            let result = verifier.verify_creation(
                register_id, 
                owner, 
                register_type, 
                initial_value, 
                &proof
            ).await?;
            
            // Handle verification result
        }
    },
    _ => { /* Handle other fact types */ }
}
```

### Register Update Verification

For register updates, the proof must demonstrate:

1. The updater has authority to modify the register
2. The state transition is valid according to register rules
3. The previous state exists and matches the claimed value

### Register Transfer Verification

For register transfers, the proof must demonstrate:

1. The sender has ownership of the register
2. The sender has authorized the transfer
3. The receiver has accepted the transfer (if required)

## ZK Proof Integration

Zero-knowledge proofs allow privacy-preserving verification of facts. The Causality system supports several ZK proof systems:

1. **Groth16**: Efficient for fixed-circuit applications
2. **Plonk**: More flexible for dynamic applications
3. **Stark**: Higher security with quantum resistance
4. **BulletProofs**: Simpler setup, suitable for range proofs

### ZK Proof Generation

```rust
// Example ZK proof generation for a register update
let inputs = {
    let mut map = HashMap::new();
    map.insert("register_id".to_string(), register_id.to_string());
    map.insert("previous_value".to_string(), previous_value.to_string());
    map.insert("new_value".to_string(), new_value.to_string());
    map.insert("updater".to_string(), updater.to_string());
    map
};

let witness = {
    let mut map = HashMap::new();
    map.insert("private_key".to_string(), private_key.to_string());
    // Other private inputs
    map
};

let zk_proof = zk_prover.generate_proof("register_update", &inputs, &witness).await?;
```

### ZK Proof Verification

```rust
// Example ZK proof verification for a register update
let public_inputs = {
    let mut map = HashMap::new();
    map.insert("register_id".to_string(), register_id.to_string());
    map.insert("previous_value".to_string(), previous_value.to_string());
    map.insert("new_value".to_string(), new_value.to_string());
    map.insert("updater".to_string(), updater.to_string());
    map
};

let result = zk_verifier.verify_proof("register_update", &public_inputs, &zk_proof).await?;
```

## Integration with FactType System

The new `FactType` system enhances proof verification by:

1. Providing type-safe access to proof data based on fact variants
2. Enabling specialized verification logic for different fact types
3. Supporting standardized proof formats across domains
4. Facilitating better error handling during verification

## Best Practices

When working with the fact verification system:

1. **Always verify proofs**: Never trust unverified facts for critical operations
2. **Use appropriate verifiers**: Select the correct verifier for each proof type
3. **Handle verification errors**: Implement proper error handling for verification failures
4. **Cache verification results**: Cache results to avoid redundant verification
5. **Validate proof freshness**: Check timestamps to ensure proofs aren't stale
6. **Maintain verifier registries**: Keep verifier registries updated with the latest verification algorithms

## Example: Complete Verification Flow

Here's a complete example of the verification flow for a register update fact:

```rust
async fn verify_register_update(
    domain_adapter: &dyn DomainAdapter,
    verifier_registry: &VerifierRegistry,
    register_id: &str,
    previous_value: &str,
    new_value: &str,
    updater: &str,
) -> Result<bool> {
    // 1. Create the fact query
    let mut params = HashMap::new();
    params.insert("register_id".to_string(), register_id.to_string());
    params.insert("previous_value".to_string(), previous_value.to_string());
    params.insert("new_value".to_string(), new_value.to_string());
    params.insert("updater".to_string(), updater.to_string());
    
    let query = FactQuery {
        domain_id: domain_adapter.domain_id().clone(),
        fact_type: "register_update".to_string(),
        parameters: params,
        block_height: None,
        block_hash: None,
        timestamp: None,
    };
    
    // 2. Observe the fact
    let fact = domain_adapter.observe_fact(query).await?;
    
    // 3. Extract proof from the fact
    match fact {
        FactType::RegisterFact(RegisterFact::RegisterUpdate {
            register_id,
            previous_value,
            new_value,
            updater,
            proof,
            ..
        }) => {
            // 4. Check if proof exists
            let proof = match proof {
                Some(p) => p,
                None => return Err(Error::MissingProof("No proof provided for register update".into())),
            };
            
            // 5. Select appropriate verifier
            let verifier = match verifier_registry.get_verifier(&proof.proof_type.to_string()) {
                Some(v) => v,
                None => return Err(Error::NoVerifier(format!("No verifier found for proof type: {:?}", proof.proof_type))),
            };
            
            // 6. Verify the proof
            let result = verifier.verify(&fact, &proof).await?;
            
            // 7. Handle verification result
            match result {
                VerificationResult::Valid => Ok(true),
                VerificationResult::Invalid(reason) => {
                    log::warn!("Register update verification failed: {}", reason);
                    Ok(false)
                },
                VerificationResult::Indeterminate => {
                    log::warn!("Register update verification was indeterminate");
                    Ok(false)
                },
            }
        },
        _ => Err(Error::UnexpectedFactType("Expected RegisterFact::RegisterUpdate".into())),
    }
}
```

## Conclusion

The proof verification workflow in the Causality system provides a robust mechanism for ensuring the validity of facts across domains. By integrating zero-knowledge proofs and other cryptographic verification techniques with the type-safe `FactType` system, Causality enables secure and privacy-preserving cross-domain operations. 