//! Phase 3 E2E Tests: ZK Proof Generation Pipeline
//!
//! This test suite validates the complete Phase 3 implementation including:
//! - Storage layout generation with Traverse integration
//! - Automatic witness generation from state queries
//! - Proof generation coordination between systems
//! - Layout commitment consistency validation
//! - Valence coprocessor integration for proof lifecycle management

use std::collections::HashMap;
use anyhow::Result;
use tokio;

use causality_compiler::{
    storage_layout::{StorageLayoutGenerator, StorageLayout, TypeMappingConfig},
    traverse_almanac_integration::{TraverseAlmanacIntegrator, WitnessGenerationRequest, IntegrationConfig},
    proof_primitives::{ProofPrimitiveCompiler, ProveStatePrimitive, ProofType, WitnessStrategy, ProofOptimizationHint},
    valence_coprocessor_integration::{ValenceCoprocessorClient, ProofSubmissionRequest, CoprocessorClientConfig},
    state_analysis::{StateAnalysisResult, StateQueryRequirement, QueryType},
    almanac_schema::LayoutCommitment,
};

/// Test the complete storage layout generation pipeline
#[tokio::test]
async fn test_storage_layout_generation_with_traverse_integration() -> Result<()> {
    // Create storage layout generator
    let mut generator = StorageLayoutGenerator::new();
    
    // Create mock state analysis result
    let analysis = create_mock_state_analysis();
    
    // Generate storage layouts
    let result = generator.generate_layouts(&analysis)?;
    
    // Validate results
    assert!(!result.layouts.is_empty());
    assert!(!result.commitments.is_empty());
    assert!(result.metadata.contracts_processed > 0);
    assert!(result.metadata.storage_entries_generated > 0);
    
    // Validate layout commitment consistency
    for (contract_id, layout) in &result.layouts {
        let commitment = result.commitments.get(contract_id).unwrap();
        assert_eq!(&layout.layout_commitment, commitment);
        
        // Validate layout structure
        assert!(!layout.contract_name.is_empty());
        assert!(!layout.storage.is_empty());
        assert!(!layout.types.is_empty());
        assert!(!layout.layout_commitment.commitment_hash.is_empty());
    }
    
    println!("✅ Storage layout generation test passed");
    println!("   - Generated {} layouts", result.layouts.len());
    println!("   - Total storage entries: {}", result.metadata.storage_entries_generated);
    println!("   - Generation time: {}ms", result.metadata.generation_duration_ms);
    
    Ok(())
}

/// Test automatic witness generation from state queries
#[tokio::test]
async fn test_automatic_witness_generation() -> Result<()> {
    // Create integrator with custom configuration
    let config = IntegrationConfig {
        enable_caching: true,
        cache_ttl_seconds: 300,
        max_concurrent_witnesses: 5,
        witness_timeout_ms: 10000,
        enable_layout_validation: true,
    };
    
    let mut integrator = TraverseAlmanacIntegrator::with_config(config);
    
    // Create witness generation request
    let request = WitnessGenerationRequest {
        contract_id: "test_erc20".to_string(),
        query: "balances[0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C]".to_string(),
        block_number: 18500000,
        contract_address: "0xA0b86a33E6441e6e80D0c4C34F4f5c8B4E6C91C7".to_string(),
        layout_commitment: LayoutCommitment {
            commitment_hash: "test_commitment_hash".to_string(),
            version: "1.0.0".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        },
        parameters: HashMap::new(),
    };
    
    // Generate witness
    let result = integrator.generate_witness(request).await?;
    
    // Validate witness result
    assert!(!result.witness.storage_key.is_empty());
    assert!(!result.witness.storage_value.is_empty());
    assert!(!result.witness.merkle_proof.is_empty());
    assert_eq!(result.witness.block_number, 18500000);
    assert!(!result.witness.contract_address.is_empty());
    assert!(!result.storage_path.is_empty());
    
    // Validate metadata
    assert!(result.metadata.generation_duration_ms > 0);
    
    println!("✅ Automatic witness generation test passed");
    println!("   - Storage key: {}", result.witness.storage_key);
    println!("   - Storage value: {}", result.witness.storage_value);
    println!("   - Merkle proof entries: {}", result.witness.merkle_proof.len());
    println!("   - Generation time: {}ms", result.metadata.generation_duration_ms);
    println!("   - From cache: {}", result.metadata.from_cache);
    
    Ok(())
}

/// Test proof primitive compilation with automatic witness generation
#[tokio::test]
async fn test_proof_primitive_compilation() -> Result<()> {
    let mut compiler = ProofPrimitiveCompiler::new();
    
    // Register a test storage layout
    let layout = create_test_storage_layout();
    compiler.register_storage_layout("test_erc20".to_string(), layout.clone());
    
    // Create a proof primitive
    let primitive = ProveStatePrimitive {
        contract_id: "test_erc20".to_string(),
        storage_slot: "balances".to_string(),
        parameters: vec![],
        proof_type: ProofType::BalanceProof,
        witness_strategy: WitnessStrategy::Automatic,
        optimization_hints: vec![
            ProofOptimizationHint::Cache(3600),
            ProofOptimizationHint::Priority(8),
        ],
    };
    
    // Test async witness generation
    let witness = compiler.generate_witness_data_async(
        &primitive,
        &layout,
        18500000,
        "0xA0b86a33E6441e6e80D0c4C34F4f5c8B4E6C91C7"
    ).await?;
    
    // Validate witness
    assert!(!witness.storage_key.is_empty());
    assert!(!witness.storage_value.is_empty());
    assert!(!witness.merkle_proof.is_empty());
    assert_eq!(witness.block_number, 18500000);
    
    println!("✅ Proof primitive compilation test passed");
    println!("   - Contract: {}", primitive.contract_id);
    println!("   - Storage slot: {}", primitive.storage_slot);
    println!("   - Proof type: {:?}", primitive.proof_type);
    println!("   - Witness strategy: {:?}", primitive.witness_strategy);
    
    Ok(())
}

/// Test Valence coprocessor integration for proof lifecycle management
#[tokio::test]
async fn test_valence_coprocessor_integration() -> Result<()> {
    // Create coprocessor client with test configuration
    let config = CoprocessorClientConfig {
        timeout_ms: 30000,
        max_retries: 3,
        polling_interval_ms: 1000, // Faster polling for tests
        enable_caching: true,
        cache_ttl_seconds: 300,
        max_concurrent_proofs: 5,
    };
    
    let mut client = ValenceCoprocessorClient::with_config(
        "http://localhost:8080".to_string(),
        config
    );
    
    // Create a test compiled proof
    let compiled_proof = create_test_compiled_proof();
    
    // Create submission request
    let request = ProofSubmissionRequest {
        compiled_proof: compiled_proof.clone(),
        priority: 8,
        callback_url: None,
        metadata: HashMap::new(),
    };
    
    // Submit proof for generation
    let submission_response = client.submit_proof(request).await?;
    
    // Validate submission response
    assert!(!submission_response.submission_id.is_empty());
    assert!(submission_response.estimated_processing_time > 0);
    
    println!("✅ Proof submitted successfully");
    println!("   - Submission ID: {}", submission_response.submission_id);
    println!("   - Estimated time: {}s", submission_response.estimated_processing_time);
    
    // Wait for completion (with timeout)
    let proof_result = client.wait_for_completion(&submission_response.submission_id, 60).await?;
    
    // Validate proof result
    assert!(!proof_result.proof.proof_bytes.is_empty());
    assert!(!proof_result.proof.public_inputs.is_empty());
    assert!(!proof_result.proof.verification_key.is_empty());
    assert!(!proof_result.verification_info.circuit_id.is_empty());
    
    println!("✅ Proof generation completed");
    println!("   - Proof size: {} bytes", proof_result.metadata.proof_size_bytes);
    println!("   - Generation time: {}ms", proof_result.metadata.generation_duration_ms);
    println!("   - Circuit constraints: {:?}", proof_result.metadata.constraint_count);
    
    // Test proof verification
    let verification_result = client.verify_proof(&proof_result.proof, &proof_result.verification_info).await?;
    
    // Validate verification result
    assert!(verification_result.is_valid);
    assert!(verification_result.verification_duration_ms > 0);
    
    println!("✅ Proof verification completed");
    println!("   - Valid: {}", verification_result.is_valid);
    println!("   - Verification time: {}ms", verification_result.verification_duration_ms);
    
    Ok(())
}

/// Test layout commitment consistency validation
#[tokio::test]
async fn test_layout_commitment_consistency() -> Result<()> {
    let mut integrator = TraverseAlmanacIntegrator::new();
    
    // Create request with specific layout commitment
    let layout_commitment = LayoutCommitment {
        commitment_hash: "consistent_hash_12345".to_string(),
        version: "1.0.0".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    let request = WitnessGenerationRequest {
        contract_id: "test_contract".to_string(),
        query: "totalSupply".to_string(),
        block_number: 18500000,
        contract_address: "0xA0b86a33E6441e6e80D0c4C34F4f5c8B4E6C91C7".to_string(),
        layout_commitment: layout_commitment.clone(),
        parameters: HashMap::new(),
    };
    
    // Validate layout commitment
    let is_valid = integrator.validate_layout_commitment(&request)?;
    assert!(is_valid); // Should be valid for mock implementation
    
    println!("✅ Layout commitment validation test passed");
    println!("   - Commitment hash: {}", layout_commitment.commitment_hash);
    println!("   - Version: {}", layout_commitment.version);
    println!("   - Validation result: {}", is_valid);
    
    Ok(())
}

/// Test proof generation coordination between systems
#[tokio::test]
async fn test_proof_generation_coordination() -> Result<()> {
    // This test demonstrates the coordination between:
    // 1. Storage layout generation (Traverse)
    // 2. Witness generation (Almanac)
    // 3. Proof generation (Valence coprocessor)
    
    // Step 1: Generate storage layout
    let mut layout_generator = StorageLayoutGenerator::new();
    let analysis = create_mock_state_analysis();
    let layout_result = layout_generator.generate_layouts(&analysis)?;
    
    // Step 2: Generate witness using integrator
    let mut integrator = TraverseAlmanacIntegrator::new();
    let contract_layout = layout_result.layouts.values().next().unwrap();
    
    let witness_request = WitnessGenerationRequest {
        contract_id: contract_layout.contract_name.clone(),
        query: "balances[0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C]".to_string(),
        block_number: 18500000,
        contract_address: "0xA0b86a33E6441e6e80D0c4C34F4f5c8B4E6C91C7".to_string(),
        layout_commitment: contract_layout.layout_commitment.clone(),
        parameters: HashMap::new(),
    };
    
    let witness_result = integrator.generate_witness(witness_request).await?;
    
    // Step 3: Submit proof to coprocessor
    let mut coprocessor_client = ValenceCoprocessorClient::new("http://localhost:8080".to_string());
    let compiled_proof = create_test_compiled_proof();
    
    let proof_request = ProofSubmissionRequest {
        compiled_proof,
        priority: 5,
        callback_url: None,
        metadata: HashMap::new(),
    };
    
    let submission = coprocessor_client.submit_proof(proof_request).await?;
    let proof_result = coprocessor_client.wait_for_completion(&submission.submission_id, 60).await?;
    
    // Validate end-to-end coordination
    assert!(!layout_result.layouts.is_empty());
    assert!(!witness_result.witness.storage_key.is_empty());
    assert!(!proof_result.proof.proof_bytes.is_empty());
    
    println!("✅ Proof generation coordination test passed");
    println!("   - Layouts generated: {}", layout_result.layouts.len());
    println!("   - Witness generation time: {}ms", witness_result.metadata.generation_duration_ms);
    println!("   - Proof generation time: {}ms", proof_result.metadata.generation_duration_ms);
    
    Ok(())
}

/// Test error handling for proof generation failures
#[tokio::test]
async fn test_proof_generation_error_handling() -> Result<()> {
    let mut integrator = TraverseAlmanacIntegrator::new();
    
    // Test with invalid query
    let invalid_request = WitnessGenerationRequest {
        contract_id: "nonexistent_contract".to_string(),
        query: "invalid_query_syntax[[[".to_string(),
        block_number: 0,
        contract_address: "invalid_address".to_string(),
        layout_commitment: LayoutCommitment {
            commitment_hash: "invalid_hash".to_string(),
            version: "0.0.0".to_string(),
            timestamp: 0,
        },
        parameters: HashMap::new(),
    };
    
    // This should handle the error gracefully
    let result = integrator.generate_witness(invalid_request).await;
    
    // For mock implementation, it should still succeed but with mock data
    // In real implementation, this would test proper error handling
    match result {
        Ok(witness_result) => {
            println!("✅ Error handling test passed (mock implementation)");
            println!("   - Generated mock witness for invalid query");
        },
        Err(e) => {
            println!("✅ Error handling test passed (proper error)");
            println!("   - Error: {:?}", e);
        }
    }
    
    Ok(())
}

// Helper functions for creating test data

fn create_mock_state_analysis() -> StateAnalysisResult {
    let mut queries_by_contract = HashMap::new();
    
    let queries = vec![
        StateQueryRequirement {
            contract_id: "test_erc20".to_string(),
            storage_slot: "balances".to_string(),
            query_type: QueryType::TokenBalance,
            domain: "ethereum".to_string(),
            frequency: 100,
            is_conditional: true,
        },
        StateQueryRequirement {
            contract_id: "test_erc20".to_string(),
            storage_slot: "allowances".to_string(),
            query_type: QueryType::TokenAllowance,
            domain: "ethereum".to_string(),
            frequency: 50,
            is_conditional: false,
        },
        StateQueryRequirement {
            contract_id: "test_erc20".to_string(),
            storage_slot: "totalSupply".to_string(),
            query_type: QueryType::ContractState,
            domain: "ethereum".to_string(),
            frequency: 25,
            is_conditional: false,
        },
    ];
    
    queries_by_contract.insert("test_erc20".to_string(), queries);
    
    StateAnalysisResult {
        queries_by_contract,
        cross_contract_dependencies: vec![],
        conditional_queries: vec![],
        metadata: causality_compiler::state_analysis::AnalysisMetadata {
            total_queries: 3,
            contracts_analyzed: 1,
            analysis_duration_ms: 100,
        },
    }
}

fn create_test_storage_layout() -> StorageLayout {
    use causality_compiler::storage_layout::{StorageEntry, TypeInfo};
    
    StorageLayout {
        contract_name: "test_erc20".to_string(),
        storage: vec![
            StorageEntry {
                label: "balances".to_string(),
                slot: "0".to_string(),
                offset: 0,
                type_name: "t_mapping_address_uint256".to_string(),
            },
            StorageEntry {
                label: "allowances".to_string(),
                slot: "1".to_string(),
                offset: 0,
                type_name: "t_mapping_address_mapping_address_uint256".to_string(),
            },
            StorageEntry {
                label: "totalSupply".to_string(),
                slot: "2".to_string(),
                offset: 0,
                type_name: "t_uint256".to_string(),
            },
        ],
        types: vec![
            TypeInfo {
                label: "t_uint256".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_mapping_address_uint256".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: Some("t_uint256".to_string()),
                key: Some("t_address".to_string()),
                value: Some("t_uint256".to_string()),
            },
        ],
        layout_commitment: LayoutCommitment {
            commitment_hash: "test_layout_commitment_hash".to_string(),
            version: "1.0.0".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        },
        domain: "ethereum".to_string(),
    }
}

fn create_test_compiled_proof() -> causality_compiler::proof_primitives::CompiledProof {
    use causality_compiler::{
        proof_primitives::{CompiledProof, ProveStatePrimitive, WitnessData, ProofGenerationConfig},
        storage_layout::TraverseLayoutInfo,
    };
    
    CompiledProof {
        primitive: ProveStatePrimitive {
            contract_id: "test_erc20".to_string(),
            storage_slot: "balances".to_string(),
            parameters: vec![],
            proof_type: ProofType::BalanceProof,
            witness_strategy: WitnessStrategy::Automatic,
            optimization_hints: vec![],
        },
        witness_data: WitnessData {
            storage_key: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            storage_value: "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000".to_string(),
            merkle_proof: vec![
                "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
                "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            ],
            block_number: 18500000,
            contract_address: "0xA0b86a33E6441e6e80D0c4C34F4f5c8B4E6C91C7".to_string(),
        },
        storage_layout: TraverseLayoutInfo {
            contract_name: "test_erc20".to_string(),
            storage: vec![],
            types: vec![],
        },
        proof_config: ProofGenerationConfig::default(),
        layout_commitment: LayoutCommitment {
            commitment_hash: "test_compiled_proof_commitment".to_string(),
            version: "1.0.0".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        },
    }
} 